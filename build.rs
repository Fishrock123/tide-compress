cfg_if::cfg_if! {
    if #[cfg(feature = "db-check")] {
        use std::collections::HashMap;
        use std::env;
        use std::path::Path;

        use async_std::fs::File;
        use async_std::io::BufWriter;
        use async_std::io::WriteExt;
        use serde::Deserialize;

        #[derive(Debug, Deserialize, Clone)]
        struct MimeInfo {
            compressible: Option<bool>,
        }

        #[async_std::main]
        async fn main() -> surf::Result<()> {

            let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
            let mut file = BufWriter::new(File::create(&path).await?);

            let mime_db: HashMap<String, MimeInfo> =
                surf::get("https://raw.githubusercontent.com/jshttp/mime-db/master/db.json")
                    .recv_json()
                    .await?;

            let mut builder = phf_codegen::Set::new();

            for (key, info) in mime_db {
                if matches!(info.compressible, Some(yes) if yes) {
                    builder.entry(key);
                }
            }

            writeln!(
                &mut file,
                "static MIME_DB: phf::Set<&'static str> = \n{};\n",
                builder.build()
            )
            .await?;

            Ok(())
        }
    } else {
        // no db-check
        fn main() {}
    }
}
