#!/bin/bash

# Add this script to cron:
# crontab -e
# MAILTO=""
# @reboot /home/pi/MakeReverseSshTunnel.sh

while true; do
   sleep 1;
   ssh -v -N -R 32003:localhost:22 tarpi@79.120.10.98 >> /home/pi/ssh_reconnect_log.txt 2>&1;
   echo "`date` New iteration of ssh connect $?" >> /home/pi/ssh_reconnect_log.txt
done
