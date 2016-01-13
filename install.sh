#!/bin/bash
set -e

THIS_DIR=$(cd "$(dirname "$0")"; pwd)
cd "$THIS_DIR"

PREFIX_FOR_LOGS="update_thermo:"
THERMO_PY_WAS_CHANGED=""

function RunVerbosely() { echo "$PREFIX_FOR_LOGS $@" ; "$@" ; }

function InstallIfNeeded {
   local INSTALL_PATH="$1"
   local SERVICE="$2"
   local RESTART="$3"
   local INSTALLED="`md5sum $INSTALL_PATH/$SERVICE | cut -d ' ' -f 1`"
   local NEW="`md5sum $THIS_DIR/$SERVICE | cut -d ' ' -f 1`"

   echo "$PREFIX_FOR_LOGS $SERVICE INSTALLED $INSTALLED"
   echo "$PREFIX_FOR_LOGS $SERVICE NEW       $NEW"

   if [ "$INSTALLED" = "$NEW" ]
   then
      echo "$PREFIX_FOR_LOGS $SERVICE not changed => skipping it"
   else
      echo "$PREFIX_FOR_LOGS $SERVICE changed => installing it"
      RunVerbosely sudo cp $SERVICE $INSTALL_PATH/$SERVICE
      RunVerbosely sudo systemctl daemon-reload

      if [ "$SERVICE" = "thermo.py" ]
      then
         RunVerbosely sudo systemctl enable thermo.service
         RunVerbosely sudo systemctl restart thermo.service
      fi

      if [ "$RESTART" = "true" ]
      then
         RunVerbosely sudo systemctl enable $SERVICE
         RunVerbosely sudo systemctl restart $SERVICE
      fi
   fi
}


git pull

# Too critical to be updated on regular basis
#InstallIfNeeded /etc/systemd/system reversessh.service true

# Deprecated
#InstallIfNeeded /home/pi thermo.py
#InstallIfNeeded /etc/systemd/system thermo.service true

#InstallIfNeeded /etc/systemd/system update_thermo.service
#InstallIfNeeded /etc/systemd/system update_thermo.timer true

InstallIfNeeded /etc/systemd/system setup_3g_4g.service
InstallIfNeeded /etc/systemd/system setup_3g_4g.timer true
InstallIfNeeded /etc/systemd/system setup_3g_4g_on_boot.timer true
