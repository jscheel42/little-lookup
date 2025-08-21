#!/bin/bash

docker run -d -p 15432:5432 -e POSTGRES_PASSWORD=docker -e POSTGRES_USER=docker -e POSTGRES_DB=little-lookup postgres:14
