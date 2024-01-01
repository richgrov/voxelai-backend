mod cloudflare_r2;
mod filesystem;
mod storage;

pub use cloudflare_r2::CloudflareR2Storage;
pub use filesystem::FileSystemStorage;
pub use storage::ObjectStorage;
