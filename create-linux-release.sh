#!/bin/bash

echo "Running cargo commands"
#cargo clean
cargo update
cargo build --release

folder_name="edcas-client"

if [ -d "$folder_name" ]; then
    echo "Folder '$folder_name' found. Removing..."
    # Remove the folder
    rm -rf "$folder_name"
    echo "Folder removed."
fi
echo "Creating folders"
mkdir "$folder_name"
mkdir "$folder_name"/logs

echo "Copying files"
cp -r graphics "$folder_name"/
cp settings-example.json "$folder_name"/settings-example.json
cp settings-example.json "$folder_name"/settings.json
cp materials.json "$folder_name"/materials.json
cp start.sh "$folder_name"/A
cp target/release/edcas-client "$folder_name"/

echo "Compressing files"
tar czf edcas-client-linux.tar.gz "$folder_name"

rm -rf "$folder_name"
mkdir "$folder_name"
mkdir "$folder_name"/etc
mkdir "$folder_name"/etc/"$folder_name"
cp settings-example.json "$folder_name"/etc/"$folder_name"/settings-example.json

mkdir "$folder_name"/usr

mkdir "$folder_name"/usr/bin
cp target/release/edcas-client "$folder_name"/usr/bin

mkdir "$folder_name"/usr/share
mkdir "$folder_name"/usr/share/"$folder_name"
cp materials.json "$folder_name"/usr/share/"$folder_name"/materials.json

mkdir "$folder_name"/usr/share/"$folder_name"/graphics
mkdir "$folder_name"/usr/share/"$folder_name"/graphics/logo
cp graphics/logo/edcas.png "$folder_name"/usr/share/"$folder_name"/graphics/logo/edcas.png
cp graphics/logo/edcas_128.png "$folder_name"/usr/share/"$folder_name"/graphics/logo/edcas_128.png

mkdir "$folder_name"/DEBIAN
cp control "$folder_name"/DEBIAN/

dpkg-deb --build edcas-client

rm -rf "$folder_name"