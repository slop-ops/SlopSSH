#!/bin/sh
echo "USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND"
ps -eo user,pid,pcpu,pmem,vsz,rss,tty,stat,start,time,comm --no-headers 2>/dev/null | head -500
