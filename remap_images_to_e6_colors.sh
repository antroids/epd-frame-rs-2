#!/bin/bash

# Check if both arguments are provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <source_directory> <output_directory>"
    exit 1
fi

SOURCE_DIR="$1"
OUTPUT_DIR="$2"

# Create the output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Loop through every file in the source directory
for input_path in "$SOURCE_DIR"/*; do

    # Check if it's a file (skips directories)
    if [ -f "$input_path" ]; then

        # Get the filename without the path
        filename=$(basename "$input_path")

        # Define the output path (changing extension to .bmp)
        # ${filename%.*} removes the original extension
        output_path="$OUTPUT_DIR/${filename%.*}.bmp"

        echo "Processing: $filename"

        # Execute the conversion command
        # Note: 'convert' is called as 'magick' in ImageMagick 7+
        convert -size 1x1 xc:black xc:white xc:yellow xc:red xc:blue xc:lime -append txt:- | \
        convert "$input_path" -background white -flatten -dither FloydSteinberg -define dither:diffusion-amount=90% -remap txt:- "$output_path"
    fi
done

echo "Done! All files processed to $OUTPUT_DIR"