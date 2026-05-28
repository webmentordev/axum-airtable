<template>
    <div class="max-w-3xl w-full m-auto">
        <h1 class="text-3xl font-bold mb-3">
            Your apps #{{ id }}
        </h1>
        <div class="grid grid-cols-2 gap-6">
        </div>
    </div>
</template>
<script lang="js" setup>
const { getToken } = useAuthToken();
const id = useRoute().params.id;
</script>