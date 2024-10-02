<script lang="ts">
    import { onMount, onDestroy  } from "svelte";
    import Board from "$lib/Board.svelte";
    import type { EventHandler } from "svelte/elements";
    import { PUBLIC_WEBSOCKET_URL } from '$env/static/public';

    let ws: WebSocket | null = null;

    let player_name: string | null = null;
    let room_joined = false;
    let form_submitted = false;
    let create_room = false;
    let public_room = true;
    let enter_code = [false, -1];
    let room_code: string | null = null;
    let rooms: any[] = [];
    let loading = true;

    onMount(() => {
        ws = new WebSocket(
            `wss://${PUBLIC_WEBSOCKET_URL}/`,
        );

        ws.addEventListener("message", function (event) {
            let data = JSON.parse(event.data);

            switch (data.opcode) {
                case 13:
                    rooms.forEach((obj) => {
                        if (data.d.id == obj.id) {
                            obj.players_amount += 1;
                        }
                    });

                    rooms = rooms;
                    break;
                case 17:
                    rooms = data.d.parties;
                    break;
                case 18:
                    rooms = rooms.concat([data.d]);
                    break;
                case 19:
                    rooms = rooms.filter((obj) => {
                        return obj.id != data.d.id;
                    });

                    break;
                case 20:
                    rooms.forEach((obj) => {
                        if (data.d.id == obj.id) {
                            obj.players_amount -= 1;
                        }
                    });

                    rooms = rooms;
                    break;
                case 21:
                    room_code = data.d.code;
                    break;
            }
        });

        ws.onopen = function () {
            loading = false;
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
                    room_code,
                },
            }),
        );

        room_joined = true;
        rooms = [];
    }

    function createRoom() {
        ws?.send(
            JSON.stringify({
                opcode: 15,
                d: {
                    player_name,
                    public: public_room,
                },
            }),
        );

        room_joined = true;
    }

    const onLeave = () => {
        getRooms();
        room_joined = false;
        room_code = null;
    };

    const handleSubmit: EventHandler<Event, HTMLFormElement> = async (
        event,
    ) => {
        event.preventDefault();

        form_submitted = true;
    };
</script>

<main class="background" class:loader-overlay={loading}>
    <section class="title">
        <h1>Tic-tac-toe</h1>
    </section>

    {#if !loading}
        <section class="display">
            {#if !form_submitted}
                <form on:submit|preventDefault={handleSubmit}>
                    <input
                        type="text"
                        bind:value={player_name}
                        placeholder="Enter your name"
                    />
                    <button type="submit" class="confirm">Enter</button>
                </form>
            {:else if create_room}
                <form
                    on:submit|preventDefault={(event) => {
                        handleSubmit(event);
                        create_room = false;
                        createRoom();
                    }}
                >
                    <input
                        type="checkbox"
                        aria-checked="true"
                        id="public"
                        bind:checked={public_room}
                    />
                    <label for="public"> It's public?</label>

                    <div style="justify-content: center; margin-top: 10px;">
                        <button
                            class="cancell"
                            on:click={() => (create_room = false)}>Back</button
                        >
                        <button type="submit" class="confirm">Enter</button>
                    </div>
                </form>
            {:else if enter_code[0]}
                <form
                    on:submit|preventDefault={(event) => {
                        handleSubmit(event);
                        joinRoom(Number(enter_code[1]));
                        enter_code = [false, -1];
                    }}
                >
                    <input
                        type="text"
                        bind:value={room_code}
                        placeholder="Enter room code"
                    />
                    <button
                        on:click={() => (enter_code = [false, -1])}
                        class="cancell">Back</button
                    >
                    <button type="submit" class="confirm">Enter</button>
                </form>
            {:else if !ws}
                Connecting...
            {:else if room_joined && player_name}
                <Board {ws} {player_name} {onLeave}></Board>

                {#if room_code}
                    <p>Room code: {room_code}</p>
                {/if}
            {:else if rooms != undefined && rooms.length > 0}
                <h2>Active rooms</h2>

                {#each rooms as room}
                    <div class="room-container">
                        <p>{room.player_name}'s room</p>
                        <p>{room.players_amount}/2</p>

                        <button
                            on:click={() => {
                                if (room.public) {
                                    joinRoom(room.id);
                                } else {
                                    enter_code = [true, room.id];
                                }
                            }}
                            disabled={room.players_amount === 2}
                            type="submit"
                            class="confirm">Join</button
                        >
                    </div>
                {/each}

                <br />

                <button on:click={() => (create_room = true)} class="confirm"
                    >Create your room</button
                >
            {:else}
                <p>There is no active room</p>

                <br />

                <button
                    on:click={() => (create_room = true)}
                    type="submit"
                    class="confirm">Create your room</button
                >
            {/if}
        </section>
    {:else}
        <span class="loader"></span>
    {/if}
</main>

<!-- Styles piece from https://github.com/javascriptacademy-stash/tic-tac-toe -->
<style>
    * {
        padding: 0;
        margin: 0;
        font-family: "Itim", cursive;
    }

    .background {
        background-color: #12181b;
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: 9999;
        padding-top: 1px;
    }

    .title {
        color: white;
        text-align: center;
        font-size: 40px;
        margin-top: 10%;
    }

    .display {
        color: white;
        font-size: 25px;
        text-align: center;
        margin-top: 1em;
        margin-bottom: 1em;
    }

    input[type="text"] {
        border: 1px solid #888;
        padding: 8px 12px;
        font-size: 16px;
        background-color: #333;
        color: #fff;
        border-radius: 5px;
        margin-bottom: 10px;
    }

    .confirm {
        background-color: #12181b;
        color: white;
        padding: 8px 20px;
        font-size: 16px;
        cursor: pointer;
        border-radius: 5px;
        border: 1px solid white;
        margin-right: 10px;
    }

    .confirm:hover {
        background-color: white;
        color: #12181b;
    }

    .cancell {
        background-color: #12181b;
        color: #fff;
        padding: 8px 20px;
        font-size: 16px;
        cursor: pointer;
        border-radius: 5px;
        border: 1px solid white;
    }

    .cancell:hover {
        background-color: red;
    }

    input[type="checkbox"] {
        width: auto;
        margin-right: 5px;
    }

    input[type="checkbox"]:checked {
        background-color: #007bff;
        border: 1px solid #007bff;
    }

    .room-container {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 10px;
        max-width: 600px;
        margin-left: auto;
        margin-right: auto;
    }

    .room-container p {
        margin: 0;
    }

    .loader-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0, 0, 0, 0.7);
        z-index: 9999;
    }

    .loader {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 48px;
        height: 48px;
        border: 5px solid #fff;
        border-bottom-color: transparent;
        border-radius: 50%;
        box-sizing: border-box;
        animation: rotation 1s linear infinite;
    }

    @keyframes rotation {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }
</style>
