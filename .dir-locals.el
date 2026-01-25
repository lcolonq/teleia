((rust-ts-mode  .
   ((eglot-workspace-configuration .
      (:rust-analyzer
        ( :cargo
          ( ;; :target "wasm32-unknown-unknown"
            :targetDir t)
          :hover
          (:show (:fields 10))))))))
