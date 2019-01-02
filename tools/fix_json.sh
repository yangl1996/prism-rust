#!/bin/bash
# Usage: ./fix_json.sh $filename $start_node_idx $end_node_idx

for i in $(seq $2 $3)
do
    orig="\"${i}\""
    new="\"${i}r\""
    sed_cmd="s/${orig}/${new}/g"
    sed -i .bak $sed_cmd $1
done
