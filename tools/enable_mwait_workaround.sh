#!/bin/bash
msr=0xc0011020
bit=$(( 1 << 9 ))

for i in $(seq 0 $(( `nproc` - 1 )))
do
    val=`sudo iotools/rdmsr $i $msr`
    val=$(( val | bit ))
    sudo iotools/wrmsr $i $msr $val
    sudo iotools/rdmsr $i $msr
done | uniq