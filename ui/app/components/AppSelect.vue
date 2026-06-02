<template>
    <select :value="props.modelValue" @change="emit('update:modelValue', $event.target.value)"
        :class="[$attrs.class, 'p-3 border border-gray-200 bg-gray-100 outline-none placeholder:text-gray-500 focus:outline-none w-full text-gray-600 text-sm rounded-xl cursor-pointer']"
        v-bind="{ ...$attrs, class: undefined }">
        <option value="" disabled v-if="props.placeholder">{{ props.placeholder }}</option>
        <slot></slot>
    </select>
</template>

<script setup>
const props = defineProps({
    modelValue: String,
    placeholder: String
})
const emit = defineEmits(['update:modelValue'])
</script>