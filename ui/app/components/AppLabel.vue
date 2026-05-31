<template>
    <label :class="[$attrs.class, 'text-gray-400 mb-1']">{{ text }}</label>
</template>
<script setup>
defineProps({
    text: {
        type: String,
        default: "Collection name"
    }
})
</script>