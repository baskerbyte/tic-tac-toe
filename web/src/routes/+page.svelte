<script lang="ts">
    import {onMount} from "svelte";
    import Board from './Board.svelte';
    import type {EventHandler} from "svelte/elements";

    let playerName = '';
    let opponentName = '';
    let isFormSubmitted = false;
    let socket: WebSocket | null = null;

    onMount(() => {
        socket = new WebSocket("ws://localhost:9002");

        socket.addEventListener('open', () => {
            console.log('WebSocket connection opened');
        });

        socket.addEventListener('message', (event) => {
            console.log('Message from server:', event);

            const data = JSON.parse(event.data);
            if (data.opcode == 13) {
                opponentName = data.d.name;
            }
        });

        socket.addEventListener('close', () => {
            console.log('WebSocket connection closed');
        });

        socket.addEventListener('error', () => {
            console.error('WebSocket error');
        });
    });

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
</script>

<main>
    <h1>Tic Tac Toe</h1>
    {#if playerName && isFormSubmitted && socket}
        {#if opponentName}
            <Board socket={socket}/>
        {:else}
            <p>Finding a match</p>
        {/if}
    {:else}
        <form on:submit|preventDefault={handleSubmit}>
            <input type="text" bind:value={playerName} placeholder="Enter your name"/>
            <button type="submit">Find Match</button>
        </form>
    {/if}
</main>