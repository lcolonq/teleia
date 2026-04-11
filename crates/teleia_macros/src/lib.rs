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
        let (enm, ty) = match self.nm.as_str() {
            "meshes" => ("Mesh", "teleia::mesh::Mesh"),
            "textures" => ("Texture", "teleia::texture::Texture"),
            "materials" => ("Material", "teleia::texture::Material"),
            "shaders" => ("Shader", "teleia::shader::Shader"),
            _ => return None,
        };
        let enums: Vec<_> = ents.iter().map(|(e, _)| e.clone()).collect();
        let edecl = format!("#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, enum_map::Enum)]
pub enum {} {{ {} }}", enm, enums.join(", "));
        let decl = format!("pub {}: enum_map::EnumMap<{}, {}>", self.nm, enm, ty);
        let inits: Vec<_> = ents.into_iter().map(|(e, exp)| format!("{}::{} => {}", enm, e, exp)).collect();
        let init = format!("{}: enum_map::enum_map!({})", self.nm, inits.join(", "));
        Some((edecl, decl, init))
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
        let (mut has_meshes, mut has_textures, mut has_materials, mut has_shaders) = (false, false, false, false);
        for dir in dirs {
            if let Ok(d) = dir {
                let nm = d.file_name().into_string().unwrap();
                fields.push(Field::new(base, &nm));
                match &*nm {
                    "meshes" => has_meshes = true,
                    "textures" => has_textures = true,
                    "materials" => has_materials = true,
                    "shaders" => has_shaders = true,
                    _ => {},
                };
            }
        }
        if !has_meshes { fields.push(Field { nm: "meshes".to_owned(), entries: HashSet::new() }); }
        if !has_textures { fields.push(Field { nm: "textures".to_owned(), entries: HashSet::new() }); }
        if !has_materials { fields.push(Field { nm: "materials".to_owned(), entries: HashSet::new() }); }
        if !has_shaders { fields.push(Field { nm: "shaders".to_owned(), entries: HashSet::new() }); }
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
        for (_, decl, _) in fdata.iter() {
            res += decl; res += ",\n";
        }
        res += "}\nimpl Assets {\npub fn new(ctx: &teleia::context::Context) -> Self {\nSelf {\n";
        for (_, _, init) in fdata.iter() {
            res += init; res += ",\n";
        }
        res += "}\n}\n}\n";
        res += "impl teleia::renderer::Assets for Assets {\n";
        res += "type Shader = Shader;\n";
        res += "fn shader(&self, i: Self::Shader) -> &teleia::shader::Shader { &self.shaders[i] }\n";
        res += "type Texture = Texture;\n";
        res += "fn texture(&self, i: Self::Texture) -> &teleia::texture::Texture { &self.textures[i] }\n";
        res += "type Material = Material;\n";
        res += "fn material(&self, i: Self::Material) -> &teleia::texture::Material { &self.materials[i] }\n";
        res += "type Mesh = Mesh;\n";
        res += "fn mesh(&self, i: Self::Mesh) -> &teleia::mesh::Mesh { &self.meshes[i] }\n";
        res += "}\n";
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
