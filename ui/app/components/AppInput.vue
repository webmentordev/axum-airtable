<template>
    <input :value="props.modelValue" @input="emit('update:modelValue', $event.target.value)" autocomplete="off"
        class="p-3 border border-gray-200 bg-gray-100 outline-none placeholder:tect-para focus:outline-none w-full text-gray-600 text-sm rounded-xl placeholder:text-gray-500">
</template>

<script setup>
const props = defineProps(['modelValue'])
const emit = defineEmits(['update:modelValue'])
</script>