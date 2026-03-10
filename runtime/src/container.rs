use std::collections::HashMap;

use bollard::Docker;
use bollard::models::{PortBinding,ContainerCreateBody,HostConfig};
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions, RemoveContainerOptions};
use common::error::OxideError;


pub async fn run_container(docker:&Docker,image:&str, env: Option<Vec<String>>)->Result<(String,u16),OxideError>{
    let mut port_bindings:HashMap<String,Option<Vec<PortBinding>>>=HashMap::new();
    port_bindings.insert("3000/tcp".to_string(), Some(vec![PortBinding{
        host_ip:Some("0.0.0.0".to_string()),
        host_port:Some("0".to_string()),
    }]));

    let exposed_ports:Vec<String>=vec!["3000/tcp".to_string()];


    let config= ContainerCreateBody{
        image:Some(image.to_string()),
        env,
        exposed_ports:Some(exposed_ports),
        host_config:Some(HostConfig{
            port_bindings:Some(port_bindings),
            memory:Some(256*1024*1024),
            nano_cpus:Some(500_000_000),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container=docker.create_container(None::<CreateContainerOptions>, config).await.map_err(|e|OxideError::Runtime(e.to_string()))?;

    docker.start_container(&container.id, None::<StartContainerOptions>).await.map_err(|e|OxideError::Runtime(e.to_string()))?;

    let details=docker.inspect_container(&container.id, None).await.map_err(|e|OxideError::Runtime(e.to_string()))?;

    let port=details.network_settings.and_then(|n|n.ports).and_then(|mut ports|ports.remove("3000/tcp")).flatten().and_then(|mut vec|vec.pop()).and_then(|b|b.host_port).and_then(|p|p.parse::<u16>().ok()).ok_or_else(||OxideError::Runtime("Failed to get the container port".into()))?;

    Ok((container.id,port))
}

pub async fn stop_container(docker: &Docker, id: &str) -> Result<(), OxideError> {
    docker.stop_container(id, None).await.map_err(|e| OxideError::Runtime(e.to_string()))?;
    Ok(())
}

pub async fn remove_container(docker: &Docker, id: &str) -> Result<(), OxideError> {
    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };
    docker.remove_container(id, Some(options)).await.map_err(|e| OxideError::Runtime(e.to_string()))?;
    Ok(())
}

pub async fn get_logs_container(docker: &Docker, id: &str) -> Result<String, OxideError> {
    use bollard::container::LogOutput;
    use bollard::query_parameters::LogsOptions;
    use futures_util::StreamExt;

    let options = LogsOptions {
        stdout: true,
        stderr: true,
        tail: "100".to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(id, Some(options));
    let mut logs = String::new();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(LogOutput::StdOut { message }) => {
                logs.push_str(&String::from_utf8_lossy(&message));
            }
            Ok(LogOutput::StdErr { message }) => {
                logs.push_str(&String::from_utf8_lossy(&message));
            }
            Ok(_) => {}
            Err(e) => return Err(OxideError::Runtime(e.to_string())),
        }
    }

    Ok(logs)
}
