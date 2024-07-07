# kernel-task-rs
An xtask to speed up Windows kernel driver development in rust.

# Requirements
1. [xtask](https://github.com/matklad/cargo-xtask)
2. A second system where the driver will run. 
3. 

# Setup
 1. Setup an SSH server with [key-based authentication](https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_keymanagement).
 2. Enable [testsigning](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/the-testsigning-boot-configuration-option) in the second system.
 3. Follow [xtask steps](https://github.com/matklad/cargo-xtask?tab=readme-ov-file#defining-xtasks) in order to get it working.
 4. Modify constants with "REQUIRED" to what they should be in your case in main.rs.


# Commands
- `cargo task build` builds the driver.
- `cargo task sign` signs the driver with a testsigning certificate. 
- `cargo task deploy` deploys the built and signed driver in the second system.
- `cargo task bsd` aka build, sign and deploy.

# Extra
Open an issue if you are experiencing, make it detailed if possible.
PRs are welcome.