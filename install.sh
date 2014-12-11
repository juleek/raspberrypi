#!/bin/bash
THIS_DIR=$(cd "$(dirname "$0")"; pwd)

set -e
set -x

PROJECT_PATH="/home/pi/raspberrypi/"


function InstallIfNeeded {
   local SERVICE="$1"
   local INSTALLED="`md5sum /etc/systemd/system/$SERVICE`"
   local NEW="`md5sum $THIS_DIR/$SERVICE`"
   
   echo "INSTALLED $INSTALLED"
   echo "NEW $NEW"

   if [ "$INSTALLED" = "$NEW" ]
   then
      echo "$SERVICE not changed => skipping it"
   else 
      echo "$SERVICE changed => installing it"
   fi
}


cd $PROJECT_PATH
git pull

InstallIfNeeded reversessh.service
InstallIfNeeded thermo.service
