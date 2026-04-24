#!/bin/sh
echo "=== OS ==="
cat /etc/os-release 2>/dev/null | head -5
echo ""
echo "=== KERNEL ==="
uname -a
echo ""
echo "=== UPTIME ==="
uptime
echo ""
echo "=== CPU ==="
lscpu 2>/dev/null | grep -E '^(Model name|Socket|Core|Thread|CPU\(s\)|CPU MHz)' | head -6
echo ""
echo "=== MEMORY ==="
free -h 2>/dev/null || vmstat -h 2>/dev/null | head -4
echo ""
echo "=== DISK ==="
df -h --total 2>/dev/null | tail -1 || df -h 2>/dev/null
echo ""
echo "=== NETWORK ==="
ip -br addr 2>/dev/null || ifconfig 2>/dev/null | grep -E '^[a-z]' | head -10
