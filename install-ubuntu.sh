apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fallocate -l 500G /swap
mkswap /swap
swapon /swap
