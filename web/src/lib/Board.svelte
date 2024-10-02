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
        }, 5000);
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
                console.log(data)
                opponent_name = data.d.name;
                my_turn = data.d.id == 1 ? false : true;
                id = +my_turn;
                match_result = null;
                squares = Array(9).fill(null);

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
        <button class="cancell" on:click={leaveRoom}>Leave Room</button>
    {:else}
        <div class="display">
            <p>{player_name} VS {opponent_name}</p>
        </div>

        <div class="container">
            {#each squares as square, i}
                <button
                    class="tile"
                    on:click={() => handleSquareClick(i)}
                    class:playerX={square === "X"}
                    class:playerO={!(square === "X") && square != null}
                >
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

        <div class="display">
            {#if my_turn && !match_result}
                <p>Your turn</p>
            {:else if !my_turn && !match_result}
                <p>{opponent_name} turn</p>{/if}
        </div>

        {#if match_result}
            <p>
                {#if (match_result === 1 && id === 1) || (match_result === 2 && id === 0)}
                    <p>You won</p>
                {:else if match_result === 3}
                    <p>Tie!</p>
                {:else}
                    <p>{opponent_name} won</p>
                {/if}
            </p>

            <button type="submit" on:click={playAgain} class="confirm"
                >Play Again</button
            >
            <button class="cancell" on:click={leaveRoom}>Leave Room</button>
        {/if}
    {/if}

    {#if error_message}
        <p>{error_message}</p>
    {/if}
</div>

<style>
    .container {
        margin: 0 auto;
        display: grid;
        grid-template-columns: 33% 33% 33%;
        grid-template-rows: 33% 33% 33%;
        max-width: 300px;
    }

    .tile {
        border: 1px solid white;
        min-width: 100px;
        min-height: 100px;
        display: flex;
        justify-content: center;
        align-items: center;
        font-size: 50px;
        cursor: pointer;
    }

    .playerX {
        color: #09c372;
    }

    .playerO {
        color: #498afb;
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
</style>
