#!/bin/bash
THIS_DIR=$(cd "$(dirname "$0")"; pwd)

set -e

PROJECT_PATH="/home/pi/raspberrypi/"


function RunVerbosely() { echo "$@" ; "$@" ; }

function InstallIfNeeded {
   local SERVICE="$1"
   local INSTALLED="`md5sum /etc/systemd/system/$SERVICE | cut -d ' ' -f 1`"
   local NEW="`md5sum $THIS_DIR/$SERVICE | cut -d ' ' -f 1`"
   
   echo "INSTALLED $INSTALLED"
   echo "NEW $NEW"

   if [ "$INSTALLED" = "$NEW" ]
   then
      echo "$SERVICE not changed => skipping it"
   else 
      echo "$SERVICE changed => installing it"
      RunVerbosely sudo systemctl stop $SERVICE
      RunVerbosely cp $SERVICE /etc/systemd/system/$SERVICE
      RunVerbosely sudo systemctl daemon-reload
      RunVerbosely sudo systemctl enable $SERVICE
      RunVerbosely sudo systemctl start $SERVICE
   fi
}


cd $PROJECT_PATH
git pull

InstallIfNeeded reversessh.service
InstallIfNeeded thermo.service
