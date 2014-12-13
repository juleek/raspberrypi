#!/bin/bash
set -e

THIS_DIR=$(cd "$(dirname "$0")"; pwd)
cd "$THIS_DIR"

PROJECT_PATH="/home/pi/raspberrypi/"


function RunVerbosely() { echo "update_thermo: $@" ; "$@" ; }

function InstallIfNeeded {
   local SERVICE="$1"
   local RESTART="$2"
   local INSTALLED="`md5sum /etc/systemd/system/$SERVICE | cut -d ' ' -f 1`"
   local NEW="`md5sum $THIS_DIR/$SERVICE | cut -d ' ' -f 1`"
   
   echo "update_thermo: INSTALLED $INSTALLED"
   echo "update_thermo: NEW $NEW"

   if [ "$INSTALLED" = "$NEW" ]
   then
      echo "update_thermo: $SERVICE not changed => skipping it"
   else 
      echo "update_thermo: $SERVICE changed => installing it"
      RunVerbosely cp $SERVICE /etc/systemd/system/$SERVICE
      RunVerbosely systemctl daemon-reload
      RunVerbosely systemctl enable $SERVICE

      if [ "$RESTART" = "true" ]
      then
         RunVerbosely systemctl restart $SERVICE
      fi
   fi
}


cd $PROJECT_PATH
git pull

InstallIfNeeded reversessh.service true
InstallIfNeeded thermo.service true
InstallIfNeeded update_thermo.service
