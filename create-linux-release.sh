#!/bin/bash

cargo clean
cargo update
cargo build --release

folder_name="edcas-client"

if [ -d "$folder_name" ]; then
    echo "Folder '$folder_name' found. Removing..."
    # Remove the folder
    rm -rf "$folder_name"
    echo "Folder removed."
fi

mkdir "$folder_name"
mkdir "$folder_name"/logs
cp -r graphics "$folder_name"/graphics
cp settings-example.json "$folder_name"/settings-example.json
cp settings-example.json "$folder_name"/settings.json
cp start.sh "$folder_name"/
cp target/release/edcas-client "$folder_name"/

tar czf edcas-client.tar.gz "$folder_name"