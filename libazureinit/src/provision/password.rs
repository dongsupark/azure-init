// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::Command;

use tracing::instrument;

use crate::{error::Error, User};

use super::ssh::update_sshd_config;

use crate::provision::PasswordProvisioner;

impl PasswordProvisioner {
    pub(crate) fn set(&self, user: &User) -> Result<(), Error> {
        match self {
            Self::Passwd => passwd(user),
            #[cfg(test)]
            Self::FakePasswd => Ok(()),
        }
    }
}

#[instrument(skip_all)]
fn passwd(user: &User) -> Result<(), Error> {
    // Update the sshd configuration to allow password authentication.
    let sshd_config_path = "/etc/ssh/sshd_config.d/50-azure-init.conf";
    if let Err(error) = update_sshd_config(sshd_config_path) {
        tracing::error!(
            ?error,
            sshd_config_path,
            "Failed to update sshd configuration for password authentication"
        );
        return Err(Error::UpdateSshdConfig);
    }
    let path_passwd = env!("PATH_PASSWD");

    if user.password.is_none() {
        let mut command = Command::new(path_passwd);
        command.arg("-d").arg(&user.name);
        crate::run(command)?;
    } else {
        // creating user with a non-empty password is not allowed.
        return Err(Error::NonEmptyPassword);
    }

    Ok(())
}
