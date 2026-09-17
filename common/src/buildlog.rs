/// Shared by `builder` and `runtime` so both can stream build/image-build
/// output line-by-line to whoever's driving a deployment, without either
/// crate depending on the other or on `controller`.
pub type LogSender = tokio::sync::mpsc::UnboundedSender<String>;
