#!/usr/bin/env sh

if [ -n "$PGPASS_FILEPATH" ]; then
    cp $PGPASS_FILEPATH $HOME/.pgpass
    chmod 0600 $HOME/.pgpass
fi

$HOME/little-lookup
