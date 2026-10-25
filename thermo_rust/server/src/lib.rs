pub mod cli;
pub mod message;
pub mod plot;
pub mod alerting;
pub mod cron;
pub mod db;
pub mod grpc;
pub mod sensor;


fn resolve_home(path: &std::path::Path) -> std::path::PathBuf {
   let path_str = path.to_string_lossy();
   if path.starts_with("~") {
      let home = std::env::var("HOME").expect("HOME environment variable not set");
      return Into::into(path_str.replacen('~', &home, 1));
   }
   Into::into(path)
}
