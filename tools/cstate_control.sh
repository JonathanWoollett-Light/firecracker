#!/bin/bash

if [[ $# -ne 1 ]];
then
  echo "Usage: $0 <cstate>"
  echo "  Where <cstate> is a number between 0 and 9; the lowest desired cstate to allow."
  exit -1
fi

CPUSYS="/sys/devices/system/cpu/"

lowest_cstate=$1 # Lowest allowed cstate

cpus=`ls -d $CPUSYS/* | grep "cpu[[:digit:]]\+$"`

re='([[:digit:]]+)$'

for cpu in $cpus;
do
  for cstate in $cpu/cpuidle/*;
  do
    [[ $cstate =~ $re ]]
    state=${BASH_REMATCH[1]}
    if [[ $state -gt $lowest_cstate ]]
    then
      sudo sh -c "echo 1 > $cstate/disable"
    else
      sudo sh -c "echo 0 > $cstate/disable"
    fi
    # Uncomment the following line for verbosity
    # echo - Set $cstate to `cat $cstate/disable`
  done
done

grep -R .  /sys/devices/system/cpu/cpu2/cpuidle/*/disable