<script lang="ts">
    import {onMount} from "svelte";
    import Board from './Board.svelte';
    import type {EventHandler} from "svelte/elements";

    let playerName = '';
    let opponentName = '';
    let opponentLeft: boolean | null = null;
    let id: number | null = null;
    let my_turn: boolean | null = null;
    let isFormSubmitted = false;
    let socket: WebSocket | null = null;
    let match_result: number | null;

    onMount(() => {
        connect()
    });

    function connect() {
        socket = new WebSocket("ws://localhost:9002");

        socket.addEventListener('open', () => {
            if (playerName) {
                playAgain();
            }
        });

        socket.addEventListener('message', (event) => {
            console.log('Message from server:', event);

            const data = JSON.parse(event.data);
            if (data.opcode == 13) {
                opponentName = data.d.name;
                my_turn = data.d.id != 0;
                if (data.d.id == 0) {
                    id = 1;
                } else {
                    id = 0;
                }
            } else if (data.opcode == 11) {
                match_result = data.d.status;
                isFormSubmitted = false;
            } else if (data.opcode == 14) {
                opponentName = '';
                isFormSubmitted = false;
                opponentLeft = true;
            }
        });

        socket.addEventListener('close', () => {
            socket = null;
            opponentName = '';
        });

        socket.addEventListener('error', () => {
            console.error('WebSocket error');
        });
    }

    const handleSubmit: EventHandler<Event, HTMLFormElement> = async (event) => {
        event.preventDefault();

        if (socket) {
            isFormSubmitted = true;
            socket.send(JSON.stringify({
                opcode: 12,
                d: {
                    name: playerName
                }
            }))
        }
    }

    function playAgain() {
        opponentName = '';

        id = null;
        my_turn = null;

        if (socket) {
            isFormSubmitted = true;
            socket.send(JSON.stringify({
                opcode: 12,
                d: {
                    name: playerName
                }
            }))
        }
    }
</script>

<main>
    <h1>Tic Tac Toe</h1>
    {#if playerName && isFormSubmitted && socket}
        {#if opponentName}
            <Board socket={socket} my_turn={my_turn}/>
        {:else}
            <p>Finding a match</p>
        {/if}
    {:else}
        {#if !socket}
            <button on:click={connect}>Reconnect</button>
        {:else if match_result}
            <p>{match_result === 1 ? (id === 0 ? "You won" : `${opponentName} won`) : "Tie!"}</p>

            <button on:click={playAgain}>Play Again</button>
        {:else}
            {#if opponentLeft}
                <p>Opponent left the match</p>
                <button on:click={playAgain}>Play Again</button>
            {:else}
                <form on:submit|preventDefault={handleSubmit}>
                    <input type="text" bind:value={playerName} placeholder="Enter your name"/>
                    <button type="submit">Find Match</button>
                </form>
            {/if}
        {/if}
    {/if}
</main>