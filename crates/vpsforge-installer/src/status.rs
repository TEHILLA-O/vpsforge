use vpsforge_core::HostFacts;

pub fn status_report(host: &HostFacts) -> String {
    let mut lines = vec![
        "VPSFORGE STATUS".into(),
        String::new(),
        format!("Host          {}", host.hostname),
        format!("OS            {} {}", host.os.distribution, host.os.version),
        format!("Kernel        {}", host.kernel),
        format!("CPU           {} vCPU", host.cpu_cores),
        format!("RAM           {:.0} GB", host.ram_gib()),
        format!("Disk          {:.0} GB {}", host.disk_gib(), host.disk_kind),
        format!(
            "GPU           {}",
            host.gpu
                .as_ref()
                .map(|g| g.model.clone())
                .unwrap_or_else(|| "Not detected".into())
        ),
        format!("Package       {}", host.pkg.as_str()),
        format!("Init          {}", host.init.as_str()),
        format!("Container     {}", host.virt.kind.as_str()),
        String::new(),
        "Software".into(),
        format!("  Docker      {}", host.software.docker.display()),
        format!("  Python      {}", host.software.python.display()),
        format!("  Node        {}", host.software.node.display()),
        format!("  PostgreSQL  {}", host.software.postgresql.display()),
        format!("  Nginx       {}", host.software.nginx.display()),
        format!("  Ollama      {}", host.software.ollama.display()),
    ];
    if !host.linux {
        lines.push(String::new());
        lines.push("Note: installs are Linux-only; detection and planning work here.".into());
    }
    lines.join("\n")
}
