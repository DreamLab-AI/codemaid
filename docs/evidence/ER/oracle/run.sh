#!/bin/bash
# usage: run.sh <case> <P|F> <fixsha> <testfile> <filter...>
S=/home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad; E=$S/er; R=$S/codemaid
n=$1; side=$2; fix=$3; tf=$4; shift 4
W=$E/wt/$n-$side
# transplant the fix's version of the test file (only the test file) into P
if [ "$side" != F ]; then git -C $R show $fix:$tf > $W/$tf; fi
tname=$(basename $tf .rs)
cd $W && CARGO_HOME=$S/cargo-home-priv CARGO_TARGET_DIR=$E/target/$n-$side cargo test -p sealmap-rust --test $tname -- "$@" > $E/oracle/$n-$side.log 2>&1
echo "exit=$?" >> $E/oracle/$n-$side.log
if [ "$side" != F ]; then git -C $W checkout -- $tf 2>/dev/null || rm -f $W/$tf; fi
