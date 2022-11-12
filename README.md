# Overview
A kafka-based generator for Risingwave. This application provides a convenient CLI interface for publishing a large volume of events directly to Apache Kafka, based on setting certain configuration options such as the number of spawned tasks, target qps and target event rate.

# Setup
There is a docker-compose file which provisions the following services for the nexmark-server to run:
- zookeeper
- kafka-ui (accessible on localhost:8080)
- kafka
- (OPTIONAL, ONLY IF SPECIFIED) nexmark-server
Additionally, the nexmark server itself can be either run inside docker, or run directly on your local machine.

## Running nexmark-server locally
First, build all the other infra by running ``` make setup-local ``` from the root nexmark-server directory. This provisions the zookeeper, kafka-ui and kafka, and exposes the kafka broker to your local computer on localhost:9092. Check the .env file at the root directory:

```
# host when running locally, use kafka1:19092 when running in docker
HOST="localhost:9092"
BASE_TOPIC="events"
AUCTION_TOPIC="events-auc"
BID_TOPIC="events-bid"
PERSON_TOPIC="events-per"
NUM_PARTITIONS=4
SEPARATE_TOPICS=true
```
Since we are running nexmark-server locally, ensure that the host is set to localhost:9092 before proceeding further, or else the events will not be published successfully to kafka. First, run ``` make install ``` at the root directory to ensure that nexmark-server is installed globally.

There are a few other environment variables here, so let's go over them one by one:
- SEPARATE_TOPICS: If set to false, events will be published only to one topic, which is defined by the BASE_TOPIC environment variable. If set to true, events will be published to AUCTION_TOPIC, PERSON_TOPIC and BID_TOPIC depending on the type of event. The proportion of each event can be controlled via a command line argument to nexmark-server. 

- NUM_PARTITIONS: The total number of partitions for each topic. The events will be published to each partition inside the topic in a round-robin manner

After setting these env variables, ensure you run ```nexmark-server -c``` from the command line. This cleans up and recreates all the topics according to the specified environment variables provided. You can access the kafka-ui at localhost:8080 to verify that the topics were created as expected. If you forget to run this command, you will likely see errors when generating events.

## Generating events
To generate events, you run the following command as below, ensuring you set the --event-rate and --max-events accordingly. If you do not set these values, they will follow the defaults as defined in the configuration:

```
gautammanek@Gautams-MacBook-Pro nexmark-server % nexmark-server --event-rate 100000 --max-events 100000 
Delivered 100000 events in 1.022s
```

In the above example, we set the event rate as 100,000 and the max events as 100,000 as well. This means that it should take around 1s to generate all the events and push them to kafka, we allow some extra leeway since the buffer needs to be flushed. As we increase the qps, you may notice a slight slowdown:

```
gautammanek@Gautams-MacBook-Pro nexmark-server % nexmark-server --event-rate 400000 --max-events 400000
Delivered 400000 events in 1.126995s
```
This is because the number of tokio tasks spawned is governed by the --num-event-generators flag. These tasks are assigned to a fixed thread pool, with tasks being yielded to the executor when waiting for an 'await' statement to complete (ie while waiting between intervals). If too few tasks are spawned, the degree of multiprogramming is reduced, since no tasks can be scheduled when yielding to the executor. Let's increase the number of generators, --num-event-generators, to increase the number of tasks spawned:

```
gautammanek@Gautams-MacBook-Pro nexmark-server % nexmark-server --event-rate 400000 --max-events 400000 --num-event-generators 5
Delivered 400000 events in 1.015285s
```

You can also set the --max-events flag to 0, to make the number of events generated unlimited.

## Dynamically adjusting event rate
The event rate set via command line flags can be adjusted by sending an API request to ```http://127.0.0.1:8000/nexmark/qps``` (localhost running on port 8000). This dynamic QPS adjustment enables you to change the event-rate on the fly, and ramps up the production rate of all threads. To keep the QPS scaling as smooth as possible, this is done on a best effort basis for each thread, so the qps adjustment may take some time to reflect. Allow some time for the kafka buffer to be flushed as well, before the change in QPS is reflected. 

Set QPS through `cURL` command:
```
curl -d '{"qps": {NEW_QPS}}' -H "Content-Type: application/json" -X POST http://localhost:8000/nexmark/qps
```

## Running nexmark-server inside docker
If you don't wish to run nexmark-server locally, you can also run nexmark-server inside docker. First, change the HOST in the .env file to "kafka1:19092", since we need the nexmark-server to connect to the kafka broker from inside docker. Then, run ``` make setup-docker-build ``` to simultaneously build the docker image for the nexmark-server and provision all the other infra. This may take a while, but should be faster when run again due to a caching layer. Once done, you should connect to the docker container using the following:

```
docker ps # get the container 
docker exec -it <CONTAINER_ID> /bin/sh # connect to the container and open the shell
nexmark-server -c # remember to re-create the topics
nexmark-server --event-rate 400000 --max-events 400000 --num-event-generators 5 # begin event generation
```

### Updating environment variables
Docker compose gets env variables loaded directly from the .env file at the root level. Ensure that you recreate the container using ``` make setup-docker ``` whenever you change the env variables, since docker-compose needs to reload the env variables into the container.

# Troubleshooting

Some of the main things to take note of include the following:

## Kafka-related errors

### Unknown topic/partition errors

Some kafka errors include unknown partiton errors, which are a result of nexmark-server publishing to a partition which does not exist. It is important to run ``` nexmark-server -c ``` every time any environment variables are changed, so as to recreate the topic inside kafka. If you are running nexmark-server inside docker, you also need to run ``` make setup-docker ``` to reload the changed env variables into the container.

### Local queue full errors

```
Error in sending event Bid(Bid { auction: 24600, bidder: 8801, price: 2503825, channel: "Baidu", url: "https://www.nexmark.com/bidu/item.htm?query=1", date_time: 1667712703212, extra: "" }): Message production error: QueueFull (Local: Queue full)
```

An error like this indicates that the local kafka producer's buffer is full, since events cannot be sent to kafka quickly enough. Increasing the number of generators --num-event-generators should fix the issue.

## Benchmarks
Run ```cargo bench``` to get benchmarks for different qps, event generators and event sizes. Feel free to add more benchmarks as needed.