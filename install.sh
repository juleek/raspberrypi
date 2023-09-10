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

      if [ "$RESTART" = "true" ]
      then
         RunVerbosely sudo systemctl enable $SERVICE
         RunVerbosely sudo systemctl restart $SERVICE
      fi
   fi
}

COMMIT_BEFORE_PULL=`git log thermo | head -n 1`
echo "COMMIT_BEFORE_PULL: $COMMIT_BEFORE_PULL"
git pull
COMMIT_AFTER_PULL=`git log thermo | head -n 1`
echo "COMMIT_AFTER_PULL: $COMMIT_AFTER_PULL"

if [ "$COMMIT_BEFORE_PULL" = "$COMMIT_AFTER_PULL" ]
then
   echo "Commits of thermo C++ programme are equal => skipping it"
else
   echo "Commits of thermo C++ programme are different => building it"
   # RunVerbosely rm -rf /home/pi/build-thermo
   RunVerbosely mkdir -p /home/pi/build-thermo
   RunVerbosely cd /home/pi/build-thermo
   RunVerbosely export CC=/usr/bin/clang
   RunVerbosely export CXX=/usr/bin/clang++
   RunVerbosely cmake /home/pi/raspberrypi/thermo
   RunVerbosely make -j VERBOSE=1
   RunVerbosely sudo systemctl restart thermo.service
   echo "thermo has been built and restarted"
fi

# Too critical to be updated on regular basis
# InstallIfNeeded /etc/systemd/system reversessh.service true
InstallIfNeeded /etc/systemd/system reversessh-swiss.service true

InstallIfNeeded /etc/systemd/system thermo.service true

InstallIfNeeded /etc/systemd/system update_thermo.service
InstallIfNeeded /etc/systemd/system update_thermo.timer true


InstallIfNeeded /etc/systemd/system setup_3g_4g.service
InstallIfNeeded /etc/systemd/system setup_3g_4g.timer true
InstallIfNeeded /etc/systemd/system setup_3g_4g_on_boot.timer true
