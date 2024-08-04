<script lang="ts">
    import { onMount } from "svelte";

    export let ws: WebSocket;
    export let player_name: string;
    export let onLeave: () => any;

    let opponent_name: string | null = null;
    let id: number | null = null;

    let my_turn: boolean | null = null;
    let squares = Array(9).fill(null);
    let error_message: string | null = null;
    let match_result: number | null = null;

    onMount(() => {
        setTimeout(function () {
            error_message = null;
        }, 10000);
    });

    ws.addEventListener("message", (event) => {
        let data = JSON.parse(event.data);

        switch (data.opcode) {
            case 10:
                squares[data.d.position] = "X";
                my_turn = true;

                break;
            case 11:
                match_result = data.d.status;
                break;
            case 13:
                opponent_name = data.d.name;
                my_turn = data.d.id == 1 ? false : true;
                id = +my_turn;
                match_result = null;
                squares = Array(9).fill(null);
                console.log(id);

                break;
            case 14:
                error_message = `${opponent_name} quit!`;
                opponent_name = null;
                id = null;
                my_turn = null;
                match_result = null;
                squares = Array(9).fill(null);

                break;
        }
    });

    function handleSquareClick(index: number) {
        if (!my_turn) {
            error_message = "Not your turn";
            return;
        }

        if (squares[index]) {
            error_message = "Position already taken";
            return;
        }

        ws.send(
            JSON.stringify({
                opcode: 10,
                d: {
                    position: index,
                },
            }),
        );

        squares[index] = "O";
        my_turn = false;
    }

    function playAgain() {
        ws.send(
            JSON.stringify({
                opcode: 22,
                d: null,
            }),
        );
    }

    function leaveRoom() {
        ws.send(
            JSON.stringify({
                opcode: 14,
                d: null,
            }),
        );

        onLeave();
    }
</script>

<div>
    {#if !opponent_name}
        <p>Waiting for player...</p>
    {:else if match_result}
        <p>
            {match_result === 1
                ? id === 0
                    ? `${opponent_name} won`
                    : "You won"
                : "Tie!"}
        </p>

        <button on:click={playAgain}>Play Again</button>
        <button on:click={leaveRoom}>Leave Room</button>
    {:else}
        <p>{player_name} VS {opponent_name}</p>

        <div class="board">
            {#each squares as square, i}
                <button on:click={() => handleSquareClick(i)}>
                    {#if square === "X"}
                        X
                    {:else if square === "O"}
                        O
                    {:else}
                        -
                    {/if}
                </button>
            {/each}
        </div>

        <p>Your turn: <b>{my_turn}</b></p>
    {/if}

    {#if error_message}
        <p>{error_message}</p>
    {/if}
</div>

<style>
    .board {
        display: grid;
        grid-template-columns: repeat(3, 100px);
        gap: 10px;
    }
</style>
