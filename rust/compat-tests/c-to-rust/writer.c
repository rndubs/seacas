/*
 * C Writer for Rust Compatibility Testing
 *
 * This program generates Exodus II files using the C libexodus library.
 * The generated files are then read and validated by Rust programs.
 *
 * Usage: ./writer <test_case_name>
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "exodusII.h"

#define OUTPUT_DIR "output"

/* Generate basic 2D mesh */
void generate_basic_2d(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;  /* Use double precision */

    /* Create file */
    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    /* Initialize */
    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated 2D mesh for Rust compatibility test",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Coordinates */
    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    /* Coordinate names */
    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    /* Element block */
    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);

    /* Connectivity */
    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate basic 3D mesh */
void generate_basic_3d(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 3;
    int num_nodes = 8;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated 3D mesh for Rust compatibility test",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Unit cube */
    double x[8] = {0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0};
    double y[8] = {0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0};
    double z[8] = {0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, z);

    char *coord_names[3] = {"x", "y", "z"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "HEX8", 1, 8, 0, 0, 0);

    int connect[8] = {1, 2, 3, 4, 5, 6, 7, 8};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate mesh with variables */
void generate_with_variables(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated mesh with variables",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);

    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* Add variables */
    int num_glo_vars = 1;
    ex_put_variable_param(exoid, EX_GLOBAL, num_glo_vars);
    char *glo_var_names[1] = {"time_value"};
    ex_put_variable_names(exoid, EX_GLOBAL, num_glo_vars, glo_var_names);

    int num_nod_vars = 1;
    ex_put_variable_param(exoid, EX_NODAL, num_nod_vars);
    char *nod_var_names[1] = {"temperature"};
    ex_put_variable_names(exoid, EX_NODAL, num_nod_vars, nod_var_names);

    /* Write 2 time steps */
    for (int step = 0; step < 2; step++) {
        double time_value = step * 0.1;
        ex_put_time(exoid, step + 1, &time_value);

        /* Global variable */
        double glo_vals[1] = {time_value};
        ex_put_var(exoid, step + 1, EX_GLOBAL, 1, 1, 1, glo_vals);

        /* Nodal variable */
        double nod_vals[4] = {
            100.0 + step * 10.0,
            110.0 + step * 10.0,
            120.0 + step * 10.0,
            130.0 + step * 10.0
        };
        ex_put_var(exoid, step + 1, EX_NODAL, 1, 1, num_nodes, nod_vals);
    }

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <test_case>\n", argv[0]);
        fprintf(stderr, "Test cases:\n");
        fprintf(stderr, "  basic_2d        - Simple 2D quad mesh\n");
        fprintf(stderr, "  basic_3d        - Simple 3D hex mesh\n");
        fprintf(stderr, "  with_variables  - Mesh with time-dependent variables\n");
        fprintf(stderr, "  all             - Generate all test cases\n");
        return 1;
    }

    /* Create output directory */
    system("mkdir -p " OUTPUT_DIR);

    const char *test_case = argv[1];
    char filename[256];

    if (strcmp(test_case, "basic_2d") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_basic_2d.exo", OUTPUT_DIR);
        generate_basic_2d(filename);
    }
    else if (strcmp(test_case, "basic_3d") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_basic_3d.exo", OUTPUT_DIR);
        generate_basic_3d(filename);
    }
    else if (strcmp(test_case, "with_variables") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_with_variables.exo", OUTPUT_DIR);
        generate_with_variables(filename);
    }
    else if (strcmp(test_case, "all") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_basic_2d.exo", OUTPUT_DIR);
        generate_basic_2d(filename);

        snprintf(filename, sizeof(filename), "%s/c_basic_3d.exo", OUTPUT_DIR);
        generate_basic_3d(filename);

        snprintf(filename, sizeof(filename), "%s/c_with_variables.exo", OUTPUT_DIR);
        generate_with_variables(filename);

        printf("\n✓ All test files generated successfully!\n");
    }
    else {
        fprintf(stderr, "Unknown test case: %s\n", test_case);
        return 1;
    }

    return 0;
}
