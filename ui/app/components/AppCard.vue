<template>
    <div class="bg-gray-100 border border-gray-200 rounded-lg relative" :title="value.title"
        v-for="(value, index) in apps" :key="index">
        <button class="absolute top-2 right-2 z-10 bg-white border border-gray-200 rounded-md p-1">
            <img src="https://api.iconify.design/mdi:dots-vertical.svg" width="18px">
        </button>
        <NuxtLink class="flex p-4" :to='`apps/${value.unique_id}`'>
            <div class="w-15 h-20 flex items-center justify-center font-black text-3xl rounded-lg bg-green-300 mr-2">
                {{ value.title[0] }}
            </div>
            <div class="flex flex-col">
                <strong class="text-gray-800 font-black">{{ value.title.length > 25 ? value.title.slice(0, 25) +
                    '...' : value.title }}</strong>
                <span class="text-gray-500 text-sm font-normal">{{ value.unique_id }}</span>
                <div class="flex items-center text-sm mt-1">
                    <img src="https://api.iconify.design/ci:users-group.svg" width="19px" class="mr-1">Members:
                    <span class="ml-1">{{ value.members_count }}</span>
                </div>
            </div>
        </NuxtLink>
    </div>
</template>

<script setup lang="js">
defineProps({
    apps: {
        type: Array
    },
});
</script>