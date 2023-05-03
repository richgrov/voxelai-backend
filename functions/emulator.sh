trap 'kill %1' SIGINT
tsc -w & firebase emulators:start
