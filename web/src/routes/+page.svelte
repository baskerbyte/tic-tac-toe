<script lang="ts">
    import { onMount } from "svelte";
    import Board from "$lib/Board.svelte";

    let ws: WebSocket | null = null;

    let player_name: string | null = "Luis";
    let room_joined = false;
    let rooms: any[] = [];

    onMount(() => {
        ws = new WebSocket(
            "wss://silver-enigma-x96r5pjpx9qh6px5-9002.app.github.dev/",
        );

        ws.addEventListener("message", function (event) {
            let data = JSON.parse(event.data);
            console.log(data);

            switch (data.opcode) {
                case 17:
                    rooms = data.d.parties;
                    break;
                case 18:
                    rooms = rooms.concat([data.d]);
                    break;
            }
        });

        ws.onopen = function () {
            getRooms();
        };
    });

    function getRooms() {
        ws?.send(
            JSON.stringify({
                opcode: 17,
                d: null,
            }),
        );
    }

    function joinRoom(id: number) {
        ws?.send(
            JSON.stringify({
                opcode: 12,
                d: {
                    player_name,
                    room_id: id,
                    room_code: null,
                },
            }),
        );

        room_joined = true;
    }

    function createRoom() {
        ws?.send(
            JSON.stringify({
                opcode: 15,
                d: {
                    player_name,
                    public: true,
                },
            }),
        );

        room_joined = true;
    }

    const onLeave = () => {
        getRooms();
        room_joined = false;
    };
</script>

<main>
    <h1>Tic-tac-toe</h1>

    {#if !ws}
        Connecting...
    {:else if room_joined}
        <Board {ws} {player_name} {onLeave}></Board>
    {:else if rooms != undefined && rooms.length > 0}
        <h2>Active rooms</h2>

        {#each rooms as room}
            <p>{room.player_name}'s room</p>
            <button on:click={() => joinRoom(room.id)}>Join</button>
        {/each}
        
        <button on:click={() => createRoom()}>Create your room</button>
    {:else}
        <p>There is no active room</p>

        <button on:click={() => createRoom()}>Create your room</button>
    {/if}
</main>
