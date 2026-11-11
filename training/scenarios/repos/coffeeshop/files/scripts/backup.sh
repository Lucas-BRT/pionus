#!/bin/sh
# Copies the stock database to backups/, with the date in the name.
set -eu
mkdir -p backups
cp stock.sqlite "backups/stock-$(date +%F).sqlite"
