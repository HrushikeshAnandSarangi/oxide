use bollard::Docker;
use bollard::query_parameters::BuildImageOptions;
use bytes::Bytes;
use common::buildlog::LogSender;
use common::error::OxideError;
use futures_util::TryStreamExt;
use http_body_util::Full;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, OxideError>;

fn tarball(path: PathBuf) -> Result<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all(".", path)
        .map_err(|e| OxideError::Runtime(e.to_string()))?;
    tar.into_inner()
        .map_err(|e| OxideError::Runtime(e.to_string()))
}

pub async fn build(
    docker: &Docker,
    tag: &str,
    context_path: PathBuf,
    log_tx: Option<&LogSender>,
) -> Result<()> {
    let options = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: Some(tag.to_string()),
        rm: true,
        ..Default::default()
    };

    let tar = tarball(context_path)?;
    let body = Full::new(Bytes::from(tar));
    let mut stream = docker.build_image(options, None, Some(http_body_util::Either::Left(body)));

    while let Some(msg) = stream
        .try_next()
        .await
        .map_err(|e| OxideError::Runtime(e.to_string()))?
    {
        if let Some(s) = msg.stream {
            let line = s.trim();
            tracing::info!("{}", line);
            if let Some(tx) = log_tx {
                let _ = tx.send(line.to_string());
            }
        }
    }

    Ok(())
}
