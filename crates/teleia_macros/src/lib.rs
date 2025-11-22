use proc_macro::{TokenStream};
use std::path::Path;
use std::collections::HashSet;
use walkdir::WalkDir;
use heck::ToUpperCamelCase;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Designator {
    parts: Vec<String>,
}
impl Designator {
    fn new(fbase: &str, path: &Path, shorten: bool) -> Self {
        let mut parts: Vec<_> = path.strip_prefix(fbase).unwrap().with_extension("").components()
            .filter_map(|c| match c {
                std::path::Component::Normal(o) => Some(o.to_str().unwrap().to_owned()),
                _ => None,
            })
            .collect();
        if shorten {
            parts.pop();
        }
        Self {
            parts
        }
    }
    fn enum_entry(&self, _fnm: &str) -> String {
        format!("{}", self.parts.join(" ").to_upper_camel_case())
    }
    fn load_expr(&self, fnm: &str) -> Option<String> {
        let i = format!("assets/{}/{}", fnm, self.parts.join("/"));
        match fnm {
            "meshes" =>
                Some(format!("teleia::mesh::Mesh::from_obj(ctx, include_bytes!(\"{}.obj\"))", i)),
            "textures" =>
                Some(format!("teleia::texture::Texture::new(ctx, include_bytes!(\"{}.png\"))", i)),
            "materials" =>
                Some(format!("teleia::texture::Material::new(ctx, include_bytes!(\"{}/color.jpg\"), include_bytes!(\"{}/normal.jpg\"))", i, i)),
            "shaders" => if self.parts.contains(&"nolib".to_owned()) {
                Some(format!("teleia::shader::Shader::new_nolib(ctx, include_str!(\"{}/vert.glsl\"), include_str!(\"{}/frag.glsl\"))", i, i))
            } else {
                Some(format!("teleia::shader::Shader::new(ctx, include_str!(\"{}/vert.glsl\"), include_str!(\"{}/frag.glsl\"))", i, i))
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Field {
    nm: String,
    entries: HashSet<Designator>,
}
impl Field {
    fn new(base: &str, nm: &str) -> Self {
        let mut entries = HashSet::new();
        let fbase = format!("{}{}", base, nm);
        for mf in WalkDir::new(&fbase) {
            if let Ok(f) = mf {
                if f.file_type().is_file() {
                    entries.insert(Designator::new(
                        &fbase, f.path(),
                        nm == "shaders" || nm == "materials",
                    ));
                }
            }
        }
        Self {
            nm: nm.to_owned(),
            entries,
        }
    }
    fn generate(&self) -> Option<(String, String, String)> {
        let mut ents = Vec::new();
        for d in self.entries.iter() {
            if let Some(exp) = d.load_expr(&self.nm) {
                ents.push((d.enum_entry(&self.nm), exp));
            }
        }
        if ents.len() > 0 {
            let (enm, ty) = match self.nm.as_str() {
                "meshes" => ("Mesh", "teleia::mesh::Mesh"),
                "textures" => ("Texture", "teleia::texture::Texture"),
                "materials" => ("Material", "teleia::texture::Material"),
                "shaders" => ("Shader", "teleia::shader::Shader"),
                _ => panic!("unknown asset type: {}", self.nm),
            };
            let enums: Vec<_> = ents.iter().map(|(e, _)| e.clone()).collect();
            let edecl = format!("#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum_map::Enum)]
pub enum {} {{ {} }}", enm, enums.join(", "));
            let decl = format!("pub {}: enum_map::EnumMap<{}, {}>", self.nm, enm, ty);
            let inits: Vec<_> = ents.into_iter().map(|(e, exp)| format!("{}::{} => {}", enm, e, exp)).collect();
            let init = format!("{}: enum_map::enum_map!({})", self.nm, inits.join(", "));
            Some((edecl, decl, init))
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AssetData {
    fields: Vec<Field>,
}
impl AssetData {
    fn new(base: &str) -> Self {
        let mut fields = Vec::new();
        let dirs = std::fs::read_dir(base).expect(&format!("failed to read assets directory: {}", base));
        for dir in dirs {
            if let Ok(d) = dir {
                fields.push(Field::new(base, &d.file_name().into_string().unwrap()));
            }
        }
        Self {
            fields,
        }
    }
    fn generate(&self) -> String {
        let mut res = String::new();
        let fdata: Vec<_> = self.fields.iter().filter_map(|f| f.generate()).collect();
        for (edecl, _, _) in fdata.iter() {
            res += edecl; res += "\n";
        }
        res += "pub struct Assets {\n";
        res += "pub font_default: teleia::font::Bitmap,\n";
        for (_, decl, _) in fdata.iter() {
            res += decl; res += ",\n";
        }
        res += "}\nimpl Assets {\npub fn new(ctx: &teleia::context::Context) -> Self {\nSelf {\n";
        res += "font_default: teleia::font::Bitmap::new(ctx),\n";
        for (_, _, init) in fdata.iter() {
            res += init; res += ",\n";
        }
        res += "}\n}\n}\n";
        res
    }
}

#[proc_macro]
pub fn generate_assets(s: TokenStream) -> TokenStream {
    let token = s.into_iter().next().expect("must pass asset base path as a string literal");
    let lit = litrs::StringLit::try_from(token).expect("argument was not a string literal");
    let base = lit.value();
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest path");
    let assets = AssetData::new(&format!("{}/{}", manifest, base));
    // println!("{}", assets.generate());
    format!("{}", assets.generate()).parse().expect("failed to parse generate_assets result")
}
