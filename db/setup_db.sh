#!/usr/bin/env bash
cd $(dirname "$0")
rm ../multi-sig.db
sqlite3 ../multi-sig.db < db.sql