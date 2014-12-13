#!/bin/bash
set -e

THIS_DIR=$(cd "$(dirname "$0")"; pwd)
cd "$THIS_DIR"

PROJECT_PATH="/home/pi/raspberrypi/"
PREFIX_FOR_LOGS="update_thermo:"


function RunVerbosely() { echo "$PREFIX_FOR_LOGS $@" ; "$@" ; }

function InstallIfNeeded {
   local SERVICE="$1"
   local RESTART="$2"
   local INSTALLED="`md5sum /etc/systemd/system/$SERVICE | cut -d ' ' -f 1`"
   local NEW="`md5sum $THIS_DIR/$SERVICE | cut -d ' ' -f 1`"
   
   echo "$PREFIX_FOR_LOGS INSTALLED $INSTALLED"
   echo "$PREFIX_FOR_LOGS NEW $NEW"

   if [ "$INSTALLED" = "$NEW" ]
   then
      echo "$PREFIX_FOR_LOGS $SERVICE not changed => skipping it"
   else 
      echo "$PREFIX_FOR_LOGS $SERVICE changed => installing it"
      RunVerbosely cp $SERVICE /etc/systemd/system/$SERVICE
      RunVerbosely systemctl daemon-reload
      RunVerbosely systemctl enable $SERVICE

      if [ "$RESTART" = "true" ]
      then
         RunVerbosely systemctl restart $SERVICE
      fi
   fi
}


git pull

InstallIfNeeded reversessh.service true

InstallIfNeeded thermo.service true

InstallIfNeeded update_thermo.service
InstallIfNeeded update_thermo.timer true

InstallIfNeeded setup_3g_4g.service
InstallIfNeeded setup_3g_4g.timer true
