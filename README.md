# kernel-task-rs
An xtask to speed up Windows kernel driver development in rust.

# What is kernel-task?
kernel-task is an xtask script that speeds up Windows kernel driver development by completing the reptetif tasks, which are compiling, signing, and deploying the driver. It accomplishes those tasks by sending remote commands to the second system using SSH.
kernel-task supports logging through a file.

# Requirements
1. Install [xtask](https://github.com/matklad/cargo-xtask).
2. A second system where the driver will run.
3. Logging into a file, if you are using VMWare follow [memn0ps instructions](https://github.com/memN0ps/matrix-rs?tab=readme-ov-file#usage) to setup serial logging.

# Setup
 1. Setup an SSH server with [key-based authentication](https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_keymanagement).
 2. Enable [testsigning](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/the-testsigning-boot-configuration-option) in the second system.
 4. Follow [xtask steps](https://github.com/matklad/cargo-xtask?tab=readme-ov-file#defining-xtasks) in order to get it working.
 5. Modify constants with "REQUIRED" to what they should be in your case in main.rs.

# Commands
- `cargo task build` builds the driver.
- `cargo task sign` signs the driver with a testsigning certificate. 
- `cargo task deploy` deploys the built and signed driver in the second system.
- `cargo task bsd` aka build, sign and deploy.

For release builds: add `release` at the end.

# Extra
Open an issue if you are experiencing, make it detailed if possible.
PRs are welcome.

# Special thanks
- https://github.com/memN0ps/ for the certificate installation and COM logging.
