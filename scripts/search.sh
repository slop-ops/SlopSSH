#!/bin/sh
if [ -z "$1" ]; then
    echo "Usage: search.sh <directory> [name_pattern] [content_pattern] [max_results]"
    exit 1
fi

DIR="$1"
NAME="$2"
CONTENT="$3"
MAX="${4:-100}"

if [ -n "$CONTENT" ]; then
    find "$DIR" -maxdepth 8 -type f ${NAME:+-name "$NAME"} -exec grep -l "$CONTENT" {} \; 2>/dev/null | head -"$MAX"
elif [ -n "$NAME" ]; then
    find "$DIR" -maxdepth 8 -type f -name "$NAME" 2>/dev/null | head -"$MAX"
else
    find "$DIR" -maxdepth 8 -type f 2>/dev/null | head -"$MAX"
fi
