<template>
    <span class="cursor-pointer">{{ record[field.title] || "" }}</span>
</template>

<script setup lang="js">
defineProps({
    record: {
        type: Object
    },
    field: {
        type: Object
    },
});
</script>