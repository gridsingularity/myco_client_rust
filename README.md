# Myco client implementation in Rust. 

Ensure that redis is running with ```brew services```.

Run
```
gsy-myco-sdk --log-level DEBUG run --setup myco_matcher --run-on-redis
```

Open another terminal tab. In gsy-e repo, in a virtual environment run
```
gsy-e run -t 60s -s 60m --setup myco_setup.external_myco --enable-external-connection --slot-length-realtime 2s
```

In a third tab, run
```
docker run --rm --name myco_client myco_client_rust web2
```

