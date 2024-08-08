# Version 0.3

## Process Flow
```mermaid
graph LR

subgraph Phone
DCIM[DCIM]
end

subgraph Worker Node 1
In_1[In]
Out_1[Out]
end

subgraph Worker Node 2
In_2[In]
Out_2[Out]
end

subgraph Buffer Node 1
Buffer_1[Buffer]
end

Phone -->|send-only| Buffer_1

Buffer_1 -->|send-only| In_1
Buffer_1 -->|send-only| In_2

In_1 -->|compress| Out_1
In_2 -->|compress| Out_2
```

## Permanent Storage 
```mermaid

graph RL

subgraph Buffer Node 1
Out_1[Out]
end

subgraph Worker Node 1
Out_2[Out]
end

subgraph Worker Node 2
Out_3[Out]
end

Out_2 <-->|sync| Out_1
Out_3 <-->|sync| Out_1
```

## Initial Setup
:information_source: Here we assume the following:
1. The network consists of one buffer node and two worker nodes.
2. The `DCIM` directory of the phone is shared with the buffer node (send-only).
3. All the nodes have the media-vault software installed and running.
4. We have the IP addresses and API keys of all the nodes.

### Step 1: Adding Devices
First we add all the worker nodes to the device list of the buffer node.

### Step 2: Buffer-to-In
1. Create a directory named `Buffer` in the `in` directory of the buffer node.
2. Share the newly created `Buffer` directory with the worker nodes (send-only).
3. (???) Accept the shared directory on the worker nodes on the `in` directory.

### Step 3: Out-to-Out
1. Create a directory named `Out` in the `out` directory of the buffer node.
2. Share the newly created `Out` directory with the worker nodes (send-and-receive).
3. (???) Accept the shared directory on the worker nodes on the `out` directory.
