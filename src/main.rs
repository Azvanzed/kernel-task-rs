use anyhow::{anyhow, Result};
use log::{debug, info, LevelFilter};
use std::{path::Path, process::Command, time::Duration};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

const PROJECT_NAME: &str = "REQUIRED";
const TARGET_DIR: &str = "target";
const VMLOGS_PATH: &str = "C:\\vmlogs.txt";
const VM_USERNAME: &str = "REQUIRED";
const VM_PRIVKEY: &str = "REQUIRED";
const VM_HOSTNAME: &str = "REQUIRED";

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let Some(task) = std::env::args().nth(1) else {
        return Err(anyhow! { "No task provided" });
    };

    let release = if let Some(a) = std::env::args().nth(2) {
        a.contains("release")
    } else {
        false
    };

    match task.as_str() {
        "build" => build(release),
        "sign" => sign(release),
        "deploy" => deploy(release),
        "bsd" => bsd(release),
        _ => Err(anyhow! { "Unknown command" }),
    }
}

fn get_target_dir(release: bool) -> Result<std::path::PathBuf> {
    let mut target_dir = std::env::current_dir()?.join(TARGET_DIR);
    if release {
        target_dir.push("release");
    } else {
        target_dir.push("debug");
    }
    Ok(target_dir)
}

fn build(release: bool) -> Result<()> {
    if release {
        info!("Building release");
    } else {
        info!("Building debug");
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = std::process::Command::new(cargo);
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    cmd.arg("--package").arg(PROJECT_NAME);

    if !cmd.status()?.success() {
        return Err(anyhow! { "Cargo build failed" });
    }
    debug!("Cargo build successful");

    // use TARGET_DIR as the target directory
    let target_dir = get_target_dir(release)?;
    debug!("Target directory: {:?}", target_dir);

    // replace and '-' with '_'
    let target_name = PROJECT_NAME.replace("-", "_");

    let mut src = target_dir.join(&target_name);
    src.set_extension("dll");
    debug!("Source file: {:?}", src);

    let mut dst = target_dir.join(&target_name);
    dst.set_extension("sys");
    debug!("Destination file: {:?}", dst);

    std::fs::rename(src, dst)?;

    info!("Build successful");
    Ok(())
}

fn transfer_file(src: &std::path::Path, dst: &str) -> Result<()> {
    let output = Command::new("scp")
        .arg("-i")
        .arg(VM_PRIVKEY)
        .arg(src)
        .arg(format!("{}@{}:{}", VM_USERNAME, VM_HOSTNAME, dst))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow! { stderr });
    }

    Ok(())
}

fn send_cmds(cmds: &[&str]) -> Result<String> {
    let cmds = cmds.join(" && ");

    let output = Command::new("ssh")
        .arg(format!("{}@{}", VM_USERNAME, VM_HOSTNAME))
        .args(&["-i", VM_PRIVKEY, &cmds])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow! { stderr });
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn send_cmd(cmd: &str) -> Result<String> {
    send_cmds(&[cmd])
}

fn sign(release: bool) -> Result<()> {
    if release {
        info!("Signing release");
    } else {
        info!("Signing debug");
    }

    let target_dir = get_target_dir(release)?;
    debug!("Target directory: {:?}", target_dir);

    let target_name = format!("{}.sys", PROJECT_NAME.replace("-", "_"));
    let target = target_dir.join(&target_name);
    debug!("Target file: {:?}", target);

    let cert = Path::new("DriverCertificate.cer");
    if cert.exists() {
        std::fs::remove_file(cert)?;
        debug!("Deleted existing certificate");
    }

    let output = Command::new("makecert")
        .args(&[
            "-r",
            "-pe",
            "-ss",
            "PrivateCertStore",
            "-n",
            "CN=DriverCertificate",
            "DriverCertificate.cer",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow! { stderr });
    }
    debug!("Created certificate");

    let output = Command::new("signtool")
        .args(&[
            "sign",
            "/a",
            "/v",
            "/s",
            "PrivateCertStore",
            "/n",
            "DriverCertificate",
            "/fd",
            "sha256",
            "/t",
            "http://timestamp.digicert.com",
            target.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow! { stderr });
    }
    debug!("Signed driver");

    std::fs::remove_file(cert)?;
    debug!("Deleted certificate");

    info!("Sign successful");

    Ok(())
}

fn deploy(release: bool) -> Result<()> {
    if release {
        info!("Deploying release");
    } else {
        info!("Deploying debug");
    }

    let target_dir = get_target_dir(release)?;
    debug!("Target directory: {:?}", target_dir);

    let target_name = format!("{}.sys", PROJECT_NAME.replace("-", "_"));
    let target = target_dir.join(&target_name);
    let target_pdb = target.with_extension("pdb");
    debug!("Target file: {:?}", target);

    // get TEMP folder from remote using send_command
    let temp_dir = send_cmds(&["echo %TEMP%"])?;
    let remote_target = format!("{}\\{}", temp_dir.trim(), target_name);

    // if service exists, delete it
    if let Ok(s) = send_cmd(&format!("sc query {}", PROJECT_NAME)) {
        if s.contains("RUNNING") {
            send_cmd(&format!("sc stop {}", PROJECT_NAME))?;
            debug!("Stopped existing service");
        } else if s.contains("STOP_PENDING") {
            info!("Waiting for service to stop");
            while send_cmd(&format!("sc query {}", PROJECT_NAME))?.contains("STOP_PENDING") {
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        send_cmd(&format!("sc delete {}", PROJECT_NAME))?;
        debug!("Deleted existing service");
    }

    // transfer the file to the remote machine
    transfer_file(&target, &temp_dir)?;
    debug!("Transferred driver file to remote machine");

    if target_pdb.exists() {
        transfer_file(&target_pdb, &temp_dir)?;
        debug!("Transferred PDB file to remote machine");
    }

    // clear the logs
    std::fs::write(VMLOGS_PATH, "")?;

    // allow logging then create and start service
    send_cmds(&[
        &format!(
            "sc create {} binpath= {} type=kernel",
            PROJECT_NAME, remote_target
        ),
        &format!("sc start {}", PROJECT_NAME),
    ])?;
    info!("Created and started service");

    // create a thread for handle_logs
    info!("Logging, press 'Q' to stop");
    let mut last_size = 0;
    while let Ok(q) = send_cmd(&format!("sc query {}", PROJECT_NAME)) {
        if q.contains("STOPPED") {
            break;
        }

        if unsafe { GetAsyncKeyState('Q' as i32) } != 0 {
            // stop and delete service then remove it from disk
            send_cmds(&[
                &format!("sc stop {}", PROJECT_NAME),
                &format!("sc delete {}", PROJECT_NAME),
                &format!("del {}", remote_target),
            ])?;

            debug!("Cleaned up");
        }

        let data = std::fs::read_to_string(&VMLOGS_PATH)?;
        if data.len() > last_size {
            print!("{}", &data[last_size..]);
            last_size = data.len();
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    info!("Deploy successful");
    Ok(())
}

fn bsd(release: bool) -> Result<()> {
    info!("Building, signing, and deploying");

    build(release)?;
    sign(release)?;
    deploy(release)?;

    info!("All tasks completed");
    Ok(())
}
