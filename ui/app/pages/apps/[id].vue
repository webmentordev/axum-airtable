<template>
    <div class="max-w-3xl w-full m-auto">
        <h1 class="text-3xl font-bold mb-3">
            Your apps #{{ app.unique_id }} - {{ app.title }}
        </h1>
        <div class="grid grid-cols-2 gap-6">
        </div>
    </div>
</template>
<script lang="js" setup>
definePageMeta({
    middleware: 'auth'
});

const { getToken } = useAuthToken();
const id = useRoute().params.id;
const app = ref({});
try {
    const { data } = await useFetch('/api/apps/app', {
        method: "POST",
        body: {
            token: getToken(),
            id: id
        }
    });
    app.value = data.value.data;
} catch (e) {
    console.log(e)
}
</script>