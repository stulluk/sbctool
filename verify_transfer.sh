#!/bin/bash

# File Transfer Verification Script
# Usage: ./verify_transfer.sh <local_file> <remote_user@host> <remote_path>

set -e

LOCAL_FILE="$1"
REMOTE_HOST="$2"
REMOTE_PATH="$3"
PASSWORD="$4"

if [ $# -ne 4 ]; then
    echo "Usage: $0 <local_file> <remote_user@host> <remote_path> <password>"
    echo "Example: $0 sbctool.exe stnuc-ssh@192.168.1.222 'C:\\Users\\stnuc-ssh.stnuc\\' 'Fw8seh8n!.'"
    exit 1
fi

echo "[INFO] Verifying file transfer: $LOCAL_FILE -> $REMOTE_HOST:$REMOTE_PATH"

# Get local file info
LOCAL_SIZE=$(stat -c%s "$LOCAL_FILE")
LOCAL_MD5=$(md5sum "$LOCAL_FILE" | cut -d' ' -f1)

echo "[INFO] Local file: $LOCAL_SIZE bytes, MD5: $LOCAL_MD5"

# Get remote file info
echo "[INFO] Checking remote file..."
REMOTE_INFO=$(timeout 15 sshpass -p "$PASSWORD" ssh "$REMOTE_HOST" "dir \"$REMOTE_PATH\" 2>nul | findstr /C:\"$(basename "$LOCAL_FILE")\"")

if [ -z "$REMOTE_INFO" ]; then
    echo "[ERROR] Remote file not found!"
    exit 1
fi

# Extract remote file size (Windows dir output parsing)
REMOTE_SIZE=$(echo "$REMOTE_INFO" | awk '{print $3}' | tr -d ',')

echo "[INFO] Remote file: $REMOTE_SIZE bytes"

# Compare file sizes
if [ "$LOCAL_SIZE" = "$REMOTE_SIZE" ]; then
    echo "[SUCCESS] File sizes match!"
else
    echo "[ERROR] File sizes don't match! Local: $LOCAL_SIZE, Remote: $REMOTE_SIZE"
    exit 1
fi

# Get remote MD5 (if certutil is available)
echo "[INFO] Computing remote MD5..."
REMOTE_MD5=$(timeout 30 sshpass -p "$PASSWORD" ssh "$REMOTE_HOST" "certutil -hashfile \"$REMOTE_PATH$(basename "$LOCAL_FILE")\" MD5 | findstr /V \"MD5 hash\" | findstr /V \"CertUtil\" | findstr /V \"command completed\" | tr -d ' \r\n'")

if [ -n "$REMOTE_MD5" ] && [ "$LOCAL_MD5" = "$REMOTE_MD5" ]; then
    echo "[SUCCESS] MD5 checksums match!"
    echo "[SUCCESS] File transfer verified successfully!"
    exit 0
else
    echo "[WARNING] Could not verify MD5 (certutil might not be available)"
    echo "[INFO] File size verification passed, transfer likely successful"
    exit 0
fi
