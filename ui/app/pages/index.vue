<template>
    <div class="max-w-3xl w-full m-auto">
        <h1 class="text-3xl font-bold mb-3">
            Your apps
        </h1>
        <div class="grid grid-cols-2 gap-6 w-full border-t border-gray-200 pt-6">
            <NuxtLink :to='`apps/${value.unique_id}`' :title="value.title" v-for="(value, index) in apps" :key="index"
                class="p-4 bg-gray-100 border border-gray-200 rounded-lg flex">
                <div
                    class="w-15 h-20 flex items-center justify-center font-black text-3xl rounded-lg bg-green-300 mr-2">
                    {{ value.title[0] }}
                </div>
                <div class="flex flex-col">
                    <strong class="text-gray-800 font-black">{{ value.title.length > 20 ? value.title.slice(0, 20) +
                        '...' : value.title }}</strong>
                    <span class="text-gray-500 text-sm font-normal">{{ value.unique_id }}</span>
                    <div class="flex items-center text-sm mt-1">
                        <img src="https://api.iconify.design/ci:users-group.svg" width="19px" class="mr-1">Members:
                        <span class="ml-1">{{ value.members_count }}</span>
                    </div>
                </div>
            </NuxtLink>
        </div>
        <button @click="logout" class="bg-main text-white py-2 px-4 rounded-lg mt-4">Logout</button>
    </div>
</template>
<script lang="js" setup>
definePageMeta({
    middleware: 'auth'
});

const { getToken, removeToken } = useAuthToken();
const apps = ref([]);
try {
    const { data } = await useFetch('/api/apps/apps', {
        method: "POST",
        body: {
            token: getToken()
        }
    });
    apps.value = data.value.data;
} catch (e) {
    console.log(e)
}

async function logout() {
    console.log("Logout");
    removeToken();
    await navigateTo('/login');
}

</script>