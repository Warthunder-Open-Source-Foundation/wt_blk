#!/usr/bin/env bash

for file in *.drawio
do
  drawio --scale 4 -x -f png "$file" -o rendered/
done