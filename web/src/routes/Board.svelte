<script lang="ts">
    let squares = Array(9).fill(null);
    export let socket: WebSocket;

    export let my_turn: boolean;
    let error_message = '';

    function handleSquareClick(index: number) {
        if (!my_turn) {
            error_message = "Not your turn";
            return;
        }

        if (squares[index]) {
            error_message = "Position already taken";
            return;
        }

        socket.send(JSON.stringify({
            opcode: 10,
            d: {x: Math.floor(index / 3), y: index % 3}
        }));

        squares[index] = 'X';
        my_turn = !my_turn;
    }

    import {onMount} from "svelte";

    onMount(() => {
        socket.addEventListener('message', handleMessageFromServer);

        setTimeout(function () {
            error_message = '';
        }, 10000)
    });

    function handleMessageFromServer(event: MessageEvent) {
        const data = JSON.parse(event.data);

        if (data.opcode == 10) {
            let {x, y} = data.d;

            squares[y + x * 3] = 'O';
            my_turn = !my_turn;
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

    {#if error_message !== ''}
        <p>{error_message}</p>
    {/if}
</div>