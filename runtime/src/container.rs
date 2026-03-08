use std::collections::HashMap;

use bollard::Docker;
use bollard::models::{PortBinding,ContainerCreateBody,HostConfig};
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions};
use common::error::OxideError;


pub async fn run_container(docker:&Docker,image:&str)->Result<(String,u16),OxideError>{
    let mut port_bindings:HashMap<String,Option<Vec<PortBinding>>>=HashMap::new();
    port_bindings.insert("3000/tcp".to_string(), Some(vec![PortBinding{
        host_ip:Some("0.0.0.0".to_string()),
        host_port:Some("0".to_string()),
    }]));

    let exposed_ports:Vec<String>=vec!["3000/tcp".to_string()];


    let config= ContainerCreateBody{
        image:Some(image.to_string()),
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
