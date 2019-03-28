#!/bin/bash

if ! [ -x "$(command -v rustc)" ]; then
    echo "install rust"
    curl https://sh.rustup.rs -sSf > /tmp/rustup.sh
    sh /tmp/rustup.sh -y
    export PATH="$HOME/.cargo/bin:$PATH"
    source "$HOME/.cargo/env"
fi

neon build --release