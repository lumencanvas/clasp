#!/bin/bash
set -euo pipefail

# DigitalOcean Marketplace image cleanup
# Based on: https://github.com/digitalocean/marketplace-partners

echo "==> Cleaning up for snapshot"

# Clear apt cache
apt-get -y autoremove
apt-get -y autoclean
apt-get -y clean

# Clear machine ID (regenerated on first boot)
truncate -s 0 /etc/machine-id
[ -f /var/lib/dbus/machine-id ] && truncate -s 0 /var/lib/dbus/machine-id

# Clear SSH host keys (regenerated on first boot)
rm -f /etc/ssh/ssh_host_*

# Clear temp and logs
rm -rf /tmp/* /var/tmp/*
find /var/log -type f -exec truncate -s 0 {} \;
rm -f /var/log/*.gz /var/log/*.[0-9] /var/log/*.old

# Clear shell history
unset HISTFILE
rm -f /root/.bash_history
history -c

# Clear cloud-init state so it runs fresh on new droplet
cloud-init clean --logs 2>/dev/null || true

echo "    Cleanup complete. Ready for snapshot."
