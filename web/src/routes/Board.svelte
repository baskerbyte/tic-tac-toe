<script lang="ts">
    let squares = Array(9).fill(null);
    export let socket: WebSocket;

    function handleSquareClick(index: number) {
        if (!squares[index]) {
            squares[index] = 'X';

            socket.send(JSON.stringify({
                opcode: 10,
                d: { x: Math.floor(index / 3), y: index % 3 }
            }))
        }
    }

    import { onMount } from "svelte";
    onMount(() => {
        socket.addEventListener('message', handleMessageFromServer);
    });

    function handleMessageFromServer(event: MessageEvent) {
        const data = JSON.parse(event.data);

        if (data.opcode === 10) {
            let { x, y } = data.d;

            squares[y + x * 3] = 'O';
        }
    }
</script>

<style>
    .board {
        display: grid;
        grid-template-columns: repeat(3, 100px);
        gap: 10px;
    }
</style>

<div class="board">
    {#each squares as square, i}
        <button on:click={() => handleSquareClick(i)}>
            {#if square === 'X'}
                X
            {:else if square === 'O'}
                O
            {:else}
                -
            {/if}
        </button>
    {/each}
</div>